import { Component } from '@angular/core';
import { PageHeaderComponent } from '@m2s2/ng-lib';
import { PageHeaderConfig } from '@m2s2/ng-lib';

@Component({
  selector: 'app-contact',
  template: `<m2s2-page-header [config]="header" />`,
  standalone: true,
  imports: [PageHeaderComponent],
})
export class ContactComponent {
  readonly header: PageHeaderConfig = {
    title: 'Contact',
    subtitle: 'Get in touch with us.',
  };
}
